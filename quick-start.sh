#!/bin/bash

# AgentMem 快速启动和验证脚本
# 使用方法: ./quick-start.sh

echo "════════════════════════════════════════════════════════════"
echo "   🚀 AgentMem 快速启动脚本"
echo "════════════════════════════════════════════════════════════"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 检查是否在正确的目录
if [ ! -d "agentmem-ui" ]; then
    echo -e "${RED}❌ 错误：请在 agentmen 目录下运行此脚本${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 启动步骤：${NC}"
echo "  1️⃣  启动后端服务器（Rust）"
echo "  2️⃣  启动前端服务器（Next.js）"
echo "  3️⃣  打开浏览器验证"
echo ""

# 询问用户
echo -e "${YELLOW}请选择操作：${NC}"
echo "  [1] 仅启动后端服务器"
echo "  [2] 仅启动前端服务器"
echo "  [3] 启动后端 + 前端（推荐）"
echo "  [4] 仅打开浏览器"
echo "  [5] 查看测试指南"
echo ""
read -p "请输入选项 [1-5]: " choice

case $choice in
    1)
        echo -e "\n${GREEN}🚀 启动后端服务器...${NC}"
        echo "正在编译并启动 agent-mem-server..."
        echo ""
        cargo run --bin agent-mem-server
        ;;
    2)
        echo -e "\n${GREEN}🚀 启动前端服务器...${NC}"
        cd agentmem-ui
        echo "正在启动 Next.js 开发服务器..."
        echo ""
        npm run dev
        ;;
    3)
        echo -e "\n${GREEN}🚀 启动后端 + 前端...${NC}"
        echo ""
        echo "请在两个独立的终端窗口中执行以下命令："
        echo ""
        echo -e "${YELLOW}终端1 - 后端服务器：${NC}"
        echo "  cd $(pwd)"
        echo "  cargo run --bin agent-mem-server"
        echo ""
        echo -e "${YELLOW}终端2 - 前端服务器：${NC}"
        echo "  cd $(pwd)/agentmem-ui"
        echo "  npm run dev"
        echo ""
        echo "等待两个服务器都启动后，按Enter键继续..."
        read -p ""
        
        echo -e "\n${GREEN}🌐 打开浏览器...${NC}"
        sleep 2
        open http://localhost:3000/admin
        
        echo -e "\n${GREEN}✅ 浏览器已打开！${NC}"
        echo "请按照 START_TESTING.md 中的指南进行验证"
        ;;
    4)
        echo -e "\n${GREEN}🌐 打开浏览器...${NC}"
        open http://localhost:3000/admin
        echo ""
        echo -e "${GREEN}✅ 已打开以下页面：${NC}"
        echo "  • Dashboard: http://localhost:3000/admin"
        echo ""
        echo "其他可用页面："
        echo "  • Chat: http://localhost:3000/admin/chat"
        echo "  • Agents: http://localhost:3000/admin/agents"
        echo "  • Memories: http://localhost:3000/admin/memories"
        echo "  • Demo: http://localhost:3000/demo"
        ;;
    5)
        echo -e "\n${GREEN}📖 打开测试指南...${NC}"
        if [ -f "START_TESTING.md" ]; then
            open START_TESTING.md
            echo "已打开 START_TESTING.md"
        else
            echo -e "${RED}❌ 找不到 START_TESTING.md${NC}"
        fi
        ;;
    *)
        echo -e "${RED}❌ 无效选项${NC}"
        exit 1
        ;;
esac

echo ""
echo "════════════════════════════════════════════════════════════"

