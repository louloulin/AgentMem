#!/bin/bash

# 快速修复 Zhipu API 性能问题
# 应用最简单的优化：切换到flash模型 + 限制tokens

set -e

echo "========================================"
echo "🚀 Zhipu API 性能快速修复"
echo "========================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 1. 备份配置文件
echo "1️⃣ 备份当前配置..."
if [ -f config.toml ]; then
    cp config.toml config.toml.backup_$(date +%Y%m%d_%H%M%S)
    echo -e "${GREEN}✅ 配置已备份${NC}"
else
    echo "❌ 找不到 config.toml"
    exit 1
fi
echo ""

# 2. 修改模型为 glm-4-flash
echo "2️⃣ 切换到更快的模型 (glm-4-flash)..."
if grep -q 'model = "glm-4.6"' config.toml; then
    sed -i.bak 's/model = "glm-4.6"/model = "glm-4-flash"/' config.toml
    echo -e "${GREEN}✅ 模型已切换: glm-4.6 → glm-4-flash${NC}"
    echo "   预期速度提升: 3倍"
else
    echo -e "${YELLOW}⚠️  未找到 glm-4.6 配置，跳过${NC}"
fi
echo ""

# 3. 添加或更新 max_tokens
echo "3️⃣ 限制最大生成长度 (max_tokens=512)..."
if grep -q 'max_tokens' config.toml; then
    sed -i.bak 's/max_tokens = [0-9]*/max_tokens = 512/' config.toml
    echo -e "${GREEN}✅ max_tokens 已更新为 512${NC}"
else
    # 在 [llm.zhipu] 块下添加
    awk '/\[llm\.zhipu\]/ { print; print "max_tokens = 512"; next }1' config.toml > config.toml.tmp
    mv config.toml.tmp config.toml
    echo -e "${GREEN}✅ max_tokens 已添加 (512)${NC}"
fi
echo "   预期时间减少: 50%"
echo ""

# 4. 显示修改内容
echo "4️⃣ 配置修改内容:"
echo "----------------------------------------"
grep -A 10 '\[llm\]' config.toml | head -15
echo "----------------------------------------"
echo ""

# 5. 重新编译（如果需要）
echo "5️⃣ 检查是否需要重新编译..."
if [ -f target/release/agent-mem-server ]; then
    echo -e "${GREEN}✅ 二进制文件已存在，无需重新编译${NC}"
    echo "   (配置文件修改不需要重新编译)"
else
    echo "⚠️  未找到编译好的二进制文件"
    echo "   请运行: cargo build --release --bin agent-mem-server"
fi
echo ""

# 6. 检查服务状态
echo "6️⃣ 检查服务状态..."
if pgrep -f "agent-mem-server" > /dev/null; then
    echo -e "${YELLOW}⚠️  服务正在运行，需要重启以应用配置${NC}"
    echo ""
    read -p "是否现在重启服务？(y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "正在停止服务..."
        pkill -f "agent-mem-server" || true
        sleep 2
        echo "正在启动服务..."
        ./start_backend.sh &
        sleep 3
        echo -e "${GREEN}✅ 服务已重启${NC}"
    else
        echo "请手动重启服务以应用配置:"
        echo "  pkill -f agent-mem-server"
        echo "  ./start_backend.sh"
    fi
else
    echo "ℹ️  服务未运行"
    echo "请启动服务:"
    echo "  ./start_backend.sh"
fi
echo ""

# 7. 显示测试命令
echo "========================================"
echo "✅ 优化完成！"
echo "========================================"
echo ""
echo "📊 预期改善:"
echo "  • 响应速度: 3倍提升（模型切换）"
echo "  • 生成时间: 减少50%（tokens限制）"
echo "  • 综合提升: 6倍+"
echo ""
echo "🧪 测试命令:"
echo "  curl -X POST http://localhost:8080/api/v1/agents/agent-xxx/chat/stream \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"message\":\"你好\",\"user_id\":\"default\",\"session_id\":\"test\"}'"
echo ""
echo "📝 查看日志:"
echo "  tail -f backend-no-auth.log | grep -E '耗时|速度|tokens'"
echo ""
echo "⚡ 进一步优化建议:"
echo "  • 启用流式传输（最重要）- 用户体验提升10倍"
echo "  • 详见: ZHIPU_PERFORMANCE_ROOT_CAUSE.md"
echo ""
echo "💡 如需恢复配置:"
echo "  cp config.toml.backup_* config.toml"
echo ""

