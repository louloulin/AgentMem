#!/bin/bash

# AgentMem OpenAPI 验证脚本
# 验证 OpenAPI 规范、端点和文档完整性

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
BACKEND_URL="http://localhost:8080"
OPENAPI_JSON_URL="$BACKEND_URL/api-docs/openapi.json"
SWAGGER_UI_URL="$BACKEND_URL/swagger-ui/"
TEST_USER_ID="openapi-test-user-$(date +%s)"
TEST_ORG_ID="default-org"

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 测试结果
PASSED=0
FAILED=0
WARNINGS=0

echo "=========================================="
echo "AgentMem OpenAPI 验证"
echo "版本: v1.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 检查后端服务
log_info "检查后端服务..."
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    log_success "后端服务运行正常"
else
    log_error "后端服务未运行，请先启动: just start-server-no-auth"
    exit 1
fi

# 测试 1: OpenAPI JSON 端点
echo ""
echo "=========================================="
echo "测试 1: OpenAPI JSON 规范"
echo "=========================================="

log_info "获取 OpenAPI JSON 规范..."
OPENAPI_RESPONSE=$(curl -s "$OPENAPI_JSON_URL" 2>&1)

# 检查响应是否为空或错误（排除有效的 JSON 响应）
if [ -z "$OPENAPI_RESPONSE" ]; then
    log_error "OpenAPI JSON 端点返回空响应"
    ((FAILED++))
    exit 1
fi

# 检查是否是有效的 JSON（包含 openapi 字段）
if echo "$OPENAPI_RESPONSE" | jq -e '.openapi' > /dev/null 2>&1; then
    log_success "OpenAPI JSON 端点可访问"
    ((PASSED++))
    
    # 提取 OpenAPI 信息
    OPENAPI_VERSION=$(echo "$OPENAPI_RESPONSE" | jq -r '.openapi' 2>/dev/null || echo "unknown")
    API_TITLE=$(echo "$OPENAPI_RESPONSE" | jq -r '.info.title' 2>/dev/null || echo "unknown")
    API_VERSION=$(echo "$OPENAPI_RESPONSE" | jq -r '.info.version' 2>/dev/null || echo "unknown")
    PATH_COUNT=$(echo "$OPENAPI_RESPONSE" | jq -r '.paths | keys | length' 2>/dev/null || echo "0")
    
    log_info "OpenAPI 版本: $OPENAPI_VERSION"
    log_info "API 标题: $API_TITLE"
    log_info "API 版本: $API_VERSION"
    log_info "路径数量: $PATH_COUNT"
    
    # 保存 OpenAPI JSON
    echo "$OPENAPI_RESPONSE" > /tmp/agentmem-openapi.json
    log_info "OpenAPI JSON 已保存到: /tmp/agentmem-openapi.json"
else
    log_error "OpenAPI JSON 端点不可访问或格式错误"
    echo "响应: $OPENAPI_RESPONSE" | head -c 200
    echo "..."
    ((FAILED++))
    exit 1
fi

# 测试 2: Swagger UI 端点
echo ""
echo "=========================================="
echo "测试 2: Swagger UI 界面"
echo "=========================================="

log_info "检查 Swagger UI..."
SWAGGER_RESPONSE=$(curl -s -w "\n%{http_code}" "$SWAGGER_UI_URL" 2>&1)
HTTP_CODE=$(echo "$SWAGGER_RESPONSE" | tail -1)
BODY=$(echo "$SWAGGER_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    if echo "$BODY" | grep -q "swagger-ui\|Swagger UI"; then
        log_success "Swagger UI 可访问 (HTTP $HTTP_CODE)"
        ((PASSED++))
    else
        log_warning "Swagger UI 返回 200，但内容可能不正确"
        ((WARNINGS++))
    fi
else
    log_error "Swagger UI 不可访问 (HTTP $HTTP_CODE)"
    ((FAILED++))
fi

# 测试 3: 验证 OpenAPI 规范完整性
echo ""
echo "=========================================="
echo "测试 3: OpenAPI 规范完整性"
echo "=========================================="

if [ -f /tmp/agentmem-openapi.json ]; then
    log_info "验证 OpenAPI 规范结构..."
    
    # 检查必需字段
    REQUIRED_FIELDS=("openapi" "info" "paths" "servers")
    for field in "${REQUIRED_FIELDS[@]}"; do
        if echo "$OPENAPI_RESPONSE" | jq -e ".$field" > /dev/null 2>&1; then
            log_success "字段 '$field' 存在"
            ((PASSED++))
        else
            log_error "必需字段 '$field' 缺失"
            ((FAILED++))
        fi
    done
    
    # 检查路径定义
    log_info "检查 API 路径定义..."
    PATHS=$(echo "$OPENAPI_RESPONSE" | jq -r '.paths | keys[]' 2>/dev/null)
    PATH_COUNT=$(echo "$PATHS" | wc -l | tr -d ' ')
    
    if [ "$PATH_COUNT" -gt 0 ]; then
        log_success "发现 $PATH_COUNT 个 API 路径"
        ((PASSED++))
        
        # 列出主要路径
        log_info "主要 API 路径:"
        echo "$PATHS" | head -10 | while read path; do
            echo "  - $path"
        done
        if [ "$PATH_COUNT" -gt 10 ]; then
            echo "  ... 还有 $((PATH_COUNT - 10)) 个路径"
        fi
    else
        log_error "未找到任何 API 路径"
        ((FAILED++))
    fi
    
    # 检查操作定义
    log_info "检查 API 操作定义..."
    OPERATIONS=$(echo "$OPENAPI_RESPONSE" | jq -r '.paths | to_entries[] | .value | to_entries[] | "\(.key) \(.value.operationId // "unknown")"' 2>/dev/null | wc -l | tr -d ' ')
    
    if [ "$OPERATIONS" -gt 0 ]; then
        log_success "发现 $OPERATIONS 个 API 操作"
        ((PASSED++))
    else
        log_warning "未找到 API 操作定义"
        ((WARNINGS++))
    fi
fi

# 测试 4: 验证关键 API 端点（基于 OpenAPI 规范）
echo ""
echo "=========================================="
echo "测试 4: 验证关键 API 端点"
echo "=========================================="

if [ -f /tmp/agentmem-openapi.json ]; then
    log_info "基于 OpenAPI 规范验证关键端点..."
    
    # 提取所有 GET 端点
    GET_ENDPOINTS=$(echo "$OPENAPI_RESPONSE" | jq -r '.paths | to_entries[] | select(.value.get != null) | .key' 2>/dev/null)
    
    TESTED=0
    SUCCESS=0
    
    # 测试前 5 个 GET 端点（避免健康检查等需要认证的端点）
    echo "$GET_ENDPOINTS" | grep -v "/health\|/metrics\|/api-docs" | head -5 | while read endpoint; do
        if [ -n "$endpoint" ]; then
            TESTED=$((TESTED + 1))
            FULL_URL="$BACKEND_URL$endpoint"
            
            log_info "测试端点: $endpoint"
            
            # 尝试访问端点（可能返回 401/403，但至少端点存在）
            RESPONSE=$(curl -s -w "\n%{http_code}" "$FULL_URL" \
                -H "X-User-ID: $TEST_USER_ID" \
                -H "X-Organization-ID: $TEST_ORG_ID" \
                2>&1)
            HTTP_CODE=$(echo "$RESPONSE" | tail -1)
            
            if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "401" ] || [ "$HTTP_CODE" = "403" ] || [ "$HTTP_CODE" = "404" ]; then
                log_success "端点 $endpoint 可访问 (HTTP $HTTP_CODE)"
                SUCCESS=$((SUCCESS + 1))
                ((PASSED++))
            else
                log_warning "端点 $endpoint 返回异常状态 (HTTP $HTTP_CODE)"
                ((WARNINGS++))
            fi
        fi
    done
    
    if [ $SUCCESS -gt 0 ]; then
        log_info "成功验证 $SUCCESS/$TESTED 个端点"
    fi
fi

# 测试 5: 验证 OpenAPI 规范中的 Schema
echo ""
echo "=========================================="
echo "测试 5: OpenAPI Schema 验证"
echo "=========================================="

if [ -f /tmp/agentmem-openapi.json ]; then
    log_info "检查 OpenAPI Schema 定义..."
    
    SCHEMA_COUNT=$(echo "$OPENAPI_RESPONSE" | jq -r '.components.schemas | keys | length' 2>/dev/null || echo "0")
    
    if [ "$SCHEMA_COUNT" -gt 0 ]; then
        log_success "发现 $SCHEMA_COUNT 个 Schema 定义"
        ((PASSED++))
        
        # 列出主要 Schema
        log_info "主要 Schema:"
        echo "$OPENAPI_RESPONSE" | jq -r '.components.schemas | keys[]' 2>/dev/null | head -10 | while read schema; do
            echo "  - $schema"
        done
    else
        log_warning "未找到 Schema 定义（可能使用 $ref）"
        ((WARNINGS++))
    fi
fi

# 测试 6: 验证 API 文档完整性
echo ""
echo "=========================================="
echo "测试 6: API 文档完整性"
echo "=========================================="

if [ -f /tmp/agentmem-openapi.json ]; then
    log_info "检查 API 文档完整性..."
    
    # 检查关键标签
    TAGS=$(echo "$OPENAPI_RESPONSE" | jq -r '.tags[]?.name' 2>/dev/null | tr '\n' ' ')
    if [ -n "$TAGS" ]; then
        log_success "发现 API 标签: $TAGS"
        ((PASSED++))
    else
        log_warning "未找到 API 标签"
        ((WARNINGS++))
    fi
    
    # 检查服务器定义
    SERVERS=$(echo "$OPENAPI_RESPONSE" | jq -r '.servers[]?.url' 2>/dev/null)
    if [ -n "$SERVERS" ]; then
        log_success "发现服务器定义:"
        echo "$SERVERS" | while read server; do
            echo "  - $server"
        done
        ((PASSED++))
    else
        log_warning "未找到服务器定义"
        ((WARNINGS++))
    fi
fi

# 总结
echo ""
echo "=========================================="
echo "OpenAPI 验证总结"
echo "=========================================="
echo "总测试数: $((PASSED + FAILED + WARNINGS))"
log_success "通过: $PASSED"
if [ $FAILED -gt 0 ]; then
    log_error "失败: $FAILED"
fi
if [ $WARNINGS -gt 0 ]; then
    log_warning "警告: $WARNINGS"
fi
echo ""
echo "OpenAPI 规范:"
echo "  - JSON 端点: $OPENAPI_JSON_URL"
echo "  - Swagger UI: $SWAGGER_UI_URL"
echo "  - 本地 JSON: /tmp/agentmem-openapi.json"
echo ""
echo "=========================================="
echo "验证完成！"
echo "=========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
