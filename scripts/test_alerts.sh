#!/bin/bash
# AgentMem 告警测试脚本
# 用于验证告警规则和通知渠道

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              AgentMem 告警测试 v1.0                          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 配置
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://localhost:9093}"
REPORT_DIR="target/alert-test-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$REPORT_DIR/alert_test_$TIMESTAMP.md"

mkdir -p "$REPORT_DIR"

# 初始化报告
cat > "$REPORT_FILE" << EOF
# AgentMem 告警测试报告

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')  
**Prometheus**: $PROMETHEUS_URL  
**Alertmanager**: $ALERTMANAGER_URL

---

## 测试概述

EOF

TESTS_PASSED=0
TESTS_FAILED=0

# 函数：测试Prometheus连接
test_prometheus_connection() {
    echo -e "${YELLOW}[1/6] 测试Prometheus连接...${NC}"
    echo ""
    
    echo "## 1. Prometheus连接测试" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    if curl -s -f "$PROMETHEUS_URL/-/healthy" > /dev/null; then
        echo -e "${GREEN}✅ Prometheus连接成功${NC}"
        echo "✅ **通过**: Prometheus正常运行" >> "$REPORT_FILE"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ Prometheus连接失败${NC}"
        echo "❌ **失败**: 无法连接到Prometheus" >> "$REPORT_FILE"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    echo "" >> "$REPORT_FILE"
}

# 函数：测试Alertmanager连接
test_alertmanager_connection() {
    echo ""
    echo -e "${YELLOW}[2/6] 测试Alertmanager连接...${NC}"
    echo ""
    
    echo "## 2. Alertmanager连接测试" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    if curl -s -f "$ALERTMANAGER_URL/-/healthy" > /dev/null; then
        echo -e "${GREEN}✅ Alertmanager连接成功${NC}"
        echo "✅ **通过**: Alertmanager正常运行" >> "$REPORT_FILE"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ Alertmanager连接失败${NC}"
        echo "❌ **失败**: 无法连接到Alertmanager" >> "$REPORT_FILE"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    echo "" >> "$REPORT_FILE"
}

# 函数：检查告警规则加载
test_alert_rules() {
    echo ""
    echo -e "${YELLOW}[3/6] 检查告警规则...${NC}"
    echo ""
    
    echo "## 3. 告警规则检查" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 获取告警规则
    RULES=$(curl -s "$PROMETHEUS_URL/api/v1/rules" | jq -r '.data.groups[].rules[].alert' 2>/dev/null)
    
    if [ -z "$RULES" ]; then
        echo -e "${RED}❌ 未找到告警规则${NC}"
        echo "❌ **失败**: 未找到任何告警规则" >> "$REPORT_FILE"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    RULE_COUNT=$(echo "$RULES" | grep -c . || echo 0)
    echo -e "${GREEN}✅ 找到 $RULE_COUNT 个告警规则${NC}"
    echo "✅ **通过**: 找到 $RULE_COUNT 个告警规则" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "**告警规则列表**:" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    echo "$RULES" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

# 函数：检查当前告警状态
test_current_alerts() {
    echo ""
    echo -e "${YELLOW}[4/6] 检查当前告警状态...${NC}"
    echo ""
    
    echo "## 4. 当前告警状态" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    ACTIVE_ALERTS=$(curl -s "$PROMETHEUS_URL/api/v1/alerts" | jq -r '.data.alerts[] | select(.state=="firing") | .labels.alertname' 2>/dev/null)
    
    if [ -z "$ACTIVE_ALERTS" ]; then
        echo -e "${GREEN}✅ 当前没有活跃告警${NC}"
        echo "✅ **正常**: 当前没有活跃告警" >> "$REPORT_FILE"
    else
        ALERT_COUNT=$(echo "$ACTIVE_ALERTS" | wc -l)
        echo -e "${YELLOW}⚠️  发现 $ALERT_COUNT 个活跃告警${NC}"
        echo "⚠️ **警告**: 发现 $ALERT_COUNT 个活跃告警" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "**活跃告警**:" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "$ACTIVE_ALERTS" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
    fi
    
    echo "" >> "$REPORT_FILE"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

# 函数：测试告警通知配置
test_alertmanager_config() {
    echo ""
    echo -e "${YELLOW}[5/6] 测试Alertmanager配置...${NC}"
    echo ""
    
    echo "## 5. Alertmanager配置测试" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 获取Alertmanager状态
    STATUS=$(curl -s "$ALERTMANAGER_URL/api/v2/status" 2>/dev/null)
    
    if [ -z "$STATUS" ]; then
        echo -e "${RED}❌ 无法获取Alertmanager状态${NC}"
        echo "❌ **失败**: 无法获取Alertmanager状态" >> "$REPORT_FILE"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    echo -e "${GREEN}✅ Alertmanager配置有效${NC}"
    echo "✅ **通过**: Alertmanager配置有效" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

# 函数：发送测试告警
test_send_alert() {
    echo ""
    echo -e "${YELLOW}[6/6] 发送测试告警...${NC}"
    echo ""
    
    echo "## 6. 测试告警发送" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 构建测试告警
    TEST_ALERT=$(cat <<EOF
[
  {
    "labels": {
      "alertname": "TestAlert",
      "severity": "warning",
      "service": "agentmem",
      "test": "true"
    },
    "annotations": {
      "summary": "This is a test alert",
      "description": "This alert is sent by the alert testing script to verify the notification system."
    },
    "startsAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "endsAt": "$(date -u -d '+5 minutes' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -v+5M +%Y-%m-%dT%H:%M:%SZ)"
  }
]
EOF
)
    
    echo "发送测试告警到Alertmanager..."
    
    RESPONSE=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$TEST_ALERT" \
        "$ALERTMANAGER_URL/api/v2/alerts" 2>&1)
    
    if echo "$RESPONSE" | grep -q "success\|202\|200"; then
        echo -e "${GREEN}✅ 测试告警发送成功${NC}"
        echo "✅ **通过**: 测试告警发送成功" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "**提示**: 请检查配置的通知渠道（邮件/Slack等）是否收到测试告警" >> "$REPORT_FILE"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ 测试告警发送失败${NC}"
        echo "❌ **失败**: 测试告警发送失败" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "**错误**: $RESPONSE" >> "$REPORT_FILE"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    echo "" >> "$REPORT_FILE"
}

# 运行所有测试
echo "开始告警系统测试..."
echo ""

test_prometheus_connection
test_alertmanager_connection
test_alert_rules
test_current_alerts
test_alertmanager_config
test_send_alert

# 生成测试摘要
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 测试摘要" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| 项目 | 结果 |" >> "$REPORT_FILE"
echo "|------|------|" >> "$REPORT_FILE"
echo "| **总测试数** | $((TESTS_PASSED + TESTS_FAILED)) |" >> "$REPORT_FILE"
echo "| **通过** | $TESTS_PASSED |" >> "$REPORT_FILE"
echo "| **失败** | $TESTS_FAILED |" >> "$REPORT_FILE"
echo "| **通过率** | $(( TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED) ))% |" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if [ $TESTS_FAILED -eq 0 ]; then
    echo "✅ **结果**: 所有测试通过" >> "$REPORT_FILE"
else
    echo "❌ **结果**: 有 $TESTS_FAILED 个测试失败" >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"

# 添加后续步骤
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 后续步骤" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "1. ✅ 验证测试告警是否被接收（检查邮箱/Slack）" >> "$REPORT_FILE"
echo "2. ✅ 确认告警路由规则正确" >> "$REPORT_FILE"
echo "3. ✅ 测试告警升级策略" >> "$REPORT_FILE"
echo "4. ✅ 验证抑制规则是否生效" >> "$REPORT_FILE"
echo "5. ✅ 更新OnCall人员配置" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 显示测试结果
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  告警测试完成！                              ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}📊 测试报告:${NC} $REPORT_FILE"
echo ""
echo -e "${GREEN}测试结果:${NC}"
echo -e "  - 总测试数: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "  - 通过: $TESTS_PASSED"
echo -e "  - 失败: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有告警测试通过！${NC}"
    echo ""
    echo -e "${YELLOW}📧 请检查配置的通知渠道是否收到测试告警${NC}"
    exit 0
else
    echo -e "${RED}❌ 有 $TESTS_FAILED 个测试失败${NC}"
    exit 1
fi

