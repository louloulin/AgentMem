#!/bin/bash

# AgentMem 极简方案 - 自动重启和验证脚本
# 警告: 此脚本会停止现有的后端和前端进程

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen"
BACKEND_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3001"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  AgentMem 极简方案 - 自动重启和验证                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# ============================================
# 第1步: 停止现有进程
# ============================================

echo -e "${YELLOW}[1/5] 停止现有进程...${NC}"

# 停止后端进程
echo -e "  停止后端进程..."
pkill -f "cargo run" 2>/dev/null || true
pkill -f "agent-mem-server" 2>/dev/null || true
sleep 2

# 停止前端进程
echo -e "  停止前端进程..."
pkill -f "next-server" 2>/dev/null || true
pkill -f "next dev" 2>/dev/null || true
sleep 2

echo -e "${GREEN}✅ 旧进程已停止${NC}"

# ============================================
# 第2步: 启动后端（后台）
# ============================================

echo -e "\n${YELLOW}[2/5] 启动后端服务...${NC}"
cd "$PROJECT_ROOT"

# 后台启动后端（指定二进制文件）
nohup cargo run --release --bin agent-mem-server > /tmp/agentmem_backend.log 2>&1 &
BACKEND_PID=$!

echo -e "  后端PID: $BACKEND_PID"
echo -e "  日志: /tmp/agentmem_backend.log"

# 等待后端启动（最多60秒）
echo -e "  等待后端启动..."
for i in {1..30}; do
    if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ 后端启动成功${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ 后端启动超时${NC}"
        echo -e "${RED}查看日志: tail -f /tmp/agentmem_backend.log${NC}"
        exit 1
    fi
    sleep 2
    echo -e "  等待中... ($i/30)"
done

# ============================================
# 第3步: 启动前端（后台）
# ============================================

echo -e "\n${YELLOW}[3/5] 启动前端服务...${NC}"
cd "$PROJECT_ROOT/agentmem-website"

# 清理缓存
rm -rf .next

# 后台启动前端
nohup npm run dev > /tmp/agentmem_frontend.log 2>&1 &
FRONTEND_PID=$!

echo -e "  前端PID: $FRONTEND_PID"
echo -e "  日志: /tmp/agentmem_frontend.log"

# 等待前端启动（最多60秒）
echo -e "  等待前端启动..."
for i in {1..30}; do
    if curl -s "$FRONTEND_URL" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ 前端启动成功${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ 前端启动超时${NC}"
        echo -e "${RED}查看日志: tail -f /tmp/agentmem_frontend.log${NC}"
        exit 1
    fi
    sleep 2
    echo -e "  等待中... ($i/30)"
done

# ============================================
# 第4步: 运行验证
# ============================================

echo -e "\n${YELLOW}[4/5] 运行验证测试...${NC}"
cd "$PROJECT_ROOT"

# 等待额外2秒确保服务稳定
sleep 2

# 运行验证脚本
bash "$PROJECT_ROOT/scripts/auto_verify_and_update.sh"
VERIFY_EXIT_CODE=$?

# ============================================
# 第5步: 显示结果
# ============================================

echo -e "\n${YELLOW}[5/5] 验证完成${NC}"

if [ $VERIFY_EXIT_CODE -eq 0 ]; then
    echo -e "\n${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  🎉 全部完成！极简方案实施成功！                           ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}服务状态:${NC}"
    echo -e "  ✅ 后端运行中 (PID: $BACKEND_PID)"
    echo -e "  ✅ 前端运行中 (PID: $FRONTEND_PID)"
    echo -e "  ✅ 验证全部通过"
    echo -e "  ✅ 文档已更新"
    echo ""
    echo -e "${BLUE}后续操作:${NC}"
    echo -e "  1. 浏览器访问: ${YELLOW}http://localhost:3001/admin/memories${NC}"
    echo -e "  2. 查看后端日志: ${YELLOW}tail -f /tmp/agentmem_backend.log${NC}"
    echo -e "  3. 查看前端日志: ${YELLOW}tail -f /tmp/agentmem_frontend.log${NC}"
    echo -e "  4. 停止服务: ${YELLOW}kill $BACKEND_PID $FRONTEND_PID${NC}"
    echo ""
else
    echo -e "\n${RED}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ⚠️  验证失败，请检查日志                                   ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}调试信息:${NC}"
    echo -e "  后端日志: tail -f /tmp/agentmem_backend.log"
    echo -e "  前端日志: tail -f /tmp/agentmem_frontend.log"
    echo -e "  停止服务: kill $BACKEND_PID $FRONTEND_PID"
    echo ""
    exit 1
fi

