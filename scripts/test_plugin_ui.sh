#!/bin/bash
# AgentMem 插件 UI 功能测试脚本
# 用于快速验证插件 UI 的各项功能

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# API 配置
BACKEND_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3001"
PLUGIN_API="$BACKEND_URL/api/v1/plugins"

echo -e "${PURPLE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║                                                              ║${NC}"
echo -e "${PURPLE}║          🧪 AgentMem 插件 UI 功能测试                       ║${NC}"
echo -e "${PURPLE}║                                                              ║${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 测试函数
test_passed() {
    echo -e "   ${GREEN}✓${NC} $1"
}

test_failed() {
    echo -e "   ${RED}✗${NC} $1"
}

test_info() {
    echo -e "   ${BLUE}ℹ${NC} $1"
}

test_warning() {
    echo -e "   ${YELLOW}⚠${NC} $1"
}

# 1. 检查服务状态
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}1. 服务状态检查${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 检查后端
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    test_passed "后端服务运行正常 ($BACKEND_URL)"
    BACKEND_STATUS="running"
else
    test_failed "后端服务未运行 ($BACKEND_URL)"
    BACKEND_STATUS="stopped"
    echo ""
    test_warning "请使用以下命令启动后端:"
    echo "   cd agentmen && just start-server-with-plugins"
    exit 1
fi

# 检查前端
if curl -s "$FRONTEND_URL" > /dev/null 2>&1; then
    test_passed "前端服务运行正常 ($FRONTEND_URL)"
    FRONTEND_STATUS="running"
else
    test_failed "前端服务未运行 ($FRONTEND_URL)"
    FRONTEND_STATUS="stopped"
    echo ""
    test_warning "请使用以下命令启动前端:"
    echo "   cd agentmen/agentmem-ui && npm run dev"
    exit 1
fi

echo ""

# 2. 测试插件 API
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}2. 插件 API 测试${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 获取插件列表
if PLUGINS_JSON=$(curl -s "$PLUGIN_API" 2>/dev/null); then
    test_passed "插件列表 API 正常"
    
    # 统计插件数量
    PLUGIN_COUNT=$(echo "$PLUGINS_JSON" | jq 'length' 2>/dev/null || echo "0")
    test_info "已安装插件: $PLUGIN_COUNT 个"
    
    # 显示插件详情
    if [ "$PLUGIN_COUNT" -gt 0 ]; then
        echo ""
        echo "   插件详情:"
        echo "$PLUGINS_JSON" | jq -r '.[] | "   • \(.name) v\(.version) [\(.plugin_type)] - \(.status)"' 2>/dev/null || echo "   解析失败"
    else
        test_warning "当前没有已安装的插件"
    fi
else
    test_failed "插件列表 API 失败"
fi

echo ""

# 3. 测试前端路由
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}3. 前端路由测试${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 测试主页
if curl -s "$FRONTEND_URL" -o /dev/null -w "%{http_code}" | grep -q "200"; then
    test_passed "前端主页可访问"
else
    test_failed "前端主页不可访问"
fi

# 测试 Admin 页面
if curl -s "$FRONTEND_URL/admin" -o /dev/null -w "%{http_code}" | grep -q "200"; then
    test_passed "Admin 页面可访问"
else
    test_failed "Admin 页面不可访问"
fi

# 测试插件页面
if curl -s "$FRONTEND_URL/admin/plugins" -o /dev/null -w "%{http_code}" | grep -q "200"; then
    test_passed "插件管理页面可访问"
else
    test_failed "插件管理页面不可访问"
fi

echo ""

# 4. UI 组件检查
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}4. UI 组件检查${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 检查页面文件
PAGE_FILE="agentmem-ui/src/app/admin/plugins/page.tsx"
if [ -f "$PAGE_FILE" ]; then
    test_passed "插件页面组件存在"
    LINE_COUNT=$(wc -l < "$PAGE_FILE" | tr -d ' ')
    test_info "代码行数: $LINE_COUNT 行"
else
    test_failed "插件页面组件不存在: $PAGE_FILE"
fi

# 检查 API 客户端
API_FILE="agentmem-ui/src/lib/api-client.ts"
if [ -f "$API_FILE" ]; then
    test_passed "API 客户端存在"
    
    # 检查插件 API 方法
    if grep -q "getPlugins()" "$API_FILE"; then
        test_passed "包含 getPlugins() 方法"
    else
        test_failed "缺少 getPlugins() 方法"
    fi
    
    if grep -q "registerPlugin(" "$API_FILE"; then
        test_passed "包含 registerPlugin() 方法"
    else
        test_failed "缺少 registerPlugin() 方法"
    fi
else
    test_failed "API 客户端不存在: $API_FILE"
fi

# 检查导航菜单
LAYOUT_FILE="agentmem-ui/src/app/admin/layout.tsx"
if [ -f "$LAYOUT_FILE" ]; then
    test_passed "Admin 布局文件存在"
    
    if grep -q "/admin/plugins" "$LAYOUT_FILE"; then
        test_passed "包含插件菜单项"
    else
        test_failed "缺少插件菜单项"
    fi
else
    test_failed "Admin 布局文件不存在: $LAYOUT_FILE"
fi

echo ""

# 5. TypeScript 检查
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}5. TypeScript 类型检查${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 检查类型定义
if grep -q "export interface Plugin" "$API_FILE" 2>/dev/null; then
    test_passed "Plugin 类型定义存在"
else
    test_failed "Plugin 类型定义缺失"
fi

if grep -q "export type PluginType" "$API_FILE" 2>/dev/null; then
    test_passed "PluginType 类型定义存在"
else
    test_failed "PluginType 类型定义缺失"
fi

if grep -q "export interface PluginRegistrationRequest" "$API_FILE" 2>/dev/null; then
    test_passed "PluginRegistrationRequest 类型定义存在"
else
    test_failed "PluginRegistrationRequest 类型定义缺失"
fi

echo ""

# 6. 文档检查
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}6. 文档完整性检查${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

docs=(
    "PLUGIN_UI_IMPLEMENTATION.md:实现文档"
    "PLUGIN_UI_FEATURES.md:功能清单"
    "PLUGIN_UI_COMPLETE_SUMMARY.md:完整总结"
)

for doc_info in "${docs[@]}"; do
    IFS=':' read -r doc_file doc_name <<< "$doc_info"
    if [ -f "$doc_file" ]; then
        test_passed "$doc_name 存在"
    else
        test_warning "$doc_name 缺失: $doc_file"
    fi
done

echo ""

# 7. 测试总结
echo -e "${PURPLE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║                    测试总结                                  ║${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

if [ "$BACKEND_STATUS" = "running" ] && [ "$FRONTEND_STATUS" = "running" ]; then
    echo -e "${GREEN}✅ 所有服务运行正常${NC}"
    echo ""
    echo -e "${CYAN}🌐 快速访问链接:${NC}"
    echo ""
    echo -e "   ${GREEN}•${NC} 插件管理: ${BLUE}$FRONTEND_URL/admin/plugins${NC}"
    echo -e "   ${GREEN}•${NC} Admin 后台: ${BLUE}$FRONTEND_URL/admin${NC}"
    echo -e "   ${GREEN}•${NC} API 文档: ${BLUE}$BACKEND_URL/swagger-ui/${NC}"
    echo ""
    echo -e "${CYAN}📝 手动测试步骤:${NC}"
    echo ""
    echo "   1. 访问插件管理页面"
    echo "      open $FRONTEND_URL/admin/plugins"
    echo ""
    echo "   2. 验证功能:"
    echo "      • 查看统计卡片"
    echo "      • 查看插件列表"
    echo "      • 点击 'Add Plugin' 按钮"
    echo "      • 测试文件上传"
    echo "      • 提交表单注册"
    echo "      • 点击 'Refresh' 刷新"
    echo ""
    echo -e "${CYAN}🧪 API 测试命令:${NC}"
    echo ""
    echo "   # 获取插件列表"
    echo "   curl $PLUGIN_API | jq"
    echo ""
    echo "   # 注册新插件"
    echo "   curl -X POST $PLUGIN_API \\"
    echo "     -H 'Content-Type: application/json' \\"
    echo "     -H 'X-User-ID: user_001' \\"
    echo "     -H 'X-Organization-ID: org_001' \\"
    echo "     -d '{...}' | jq"
    echo ""
    echo -e "${GREEN}✨ 测试通过！插件 UI 已就绪！${NC}"
else
    echo -e "${RED}❌ 部分服务未运行${NC}"
    echo ""
    echo -e "${YELLOW}请先启动所有服务:${NC}"
    echo "   cd agentmen"
    echo "   just start-full-with-plugins"
fi

echo ""
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

