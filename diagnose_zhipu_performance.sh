#!/bin/bash

# 诊断Zhipu API性能问题
# 这个脚本会检查网络连接质量和API响应时间

set -e

echo "========================================"
echo "🔍 Zhipu API 性能诊断"
echo "========================================"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. 检查DNS解析时间
echo "1️⃣ DNS解析测试"
echo "----------------------------------------"
DOMAIN="open.bigmodel.cn"
DNS_START=$(date +%s%N)
DNS_RESULT=$(dig +short $DOMAIN | head -1)
DNS_END=$(date +%s%N)
DNS_TIME=$(echo "scale=3; ($DNS_END - $DNS_START) / 1000000" | bc)

if [ -n "$DNS_RESULT" ]; then
    echo -e "${GREEN}✅ DNS解析成功${NC}"
    echo "   IP地址: $DNS_RESULT"
    echo "   耗时: ${DNS_TIME}ms"
else
    echo -e "${RED}❌ DNS解析失败${NC}"
fi
echo ""

# 2. 检查网络连通性
echo "2️⃣ 网络连通性测试"
echo "----------------------------------------"
PING_RESULT=$(ping -c 5 $DOMAIN 2>&1 | grep 'avg' | awk -F'/' '{print $5}')
if [ -n "$PING_RESULT" ]; then
    echo -e "${GREEN}✅ 网络连通${NC}"
    echo "   平均延迟: ${PING_RESULT}ms"
    
    # 判断延迟等级
    PING_INT=$(echo $PING_RESULT | cut -d'.' -f1)
    if [ "$PING_INT" -lt 50 ]; then
        echo -e "   ${GREEN}延迟评级: 优秀${NC}"
    elif [ "$PING_INT" -lt 100 ]; then
        echo -e "   ${YELLOW}延迟评级: 良好${NC}"
    elif [ "$PING_INT" -lt 200 ]; then
        echo -e "   ${YELLOW}延迟评级: 一般${NC}"
    else
        echo -e "   ${RED}延迟评级: 较差${NC}"
    fi
else
    echo -e "${RED}❌ 网络不通${NC}"
fi
echo ""

# 3. 测试TLS握手时间
echo "3️⃣ TLS握手测试"
echo "----------------------------------------"
TLS_TIME=$(curl -w "@-" -o /dev/null -s "https://$DOMAIN" <<'EOF'
time_namelookup:  %{time_namelookup}s\n
time_connect:     %{time_connect}s\n
time_appconnect:  %{time_appconnect}s\n
time_pretransfer: %{time_pretransfer}s\n
time_starttransfer: %{time_starttransfer}s\n
time_total:       %{time_total}s\n
EOF
)
echo "$TLS_TIME"
echo ""

# 4. 测试简单的API调用
echo "4️⃣ Zhipu API 简单调用测试"
echo "----------------------------------------"

# 读取API key
API_KEY=$(grep 'api_key =' config.toml | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$API_KEY" ]; then
    echo -e "${RED}❌ 未找到API密钥${NC}"
    echo "   请检查 config.toml 文件"
else
    echo "正在测试API调用..."
    
    # 创建临时请求文件
    cat > /tmp/zhipu_test_request.json <<EOF
{
    "model": "glm-4-flash",
    "messages": [
        {
            "role": "user",
            "content": "你好"
        }
    ],
    "temperature": 0.7,
    "max_tokens": 10
}
EOF

    # 记录开始时间
    START_TIME=$(date +%s%N)
    
    # 发送请求并记录详细时间
    CURL_OUTPUT=$(curl -w "@-" -o /tmp/zhipu_test_response.json -s \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d @/tmp/zhipu_test_request.json \
        https://open.bigmodel.cn/api/paas/v4/chat/completions <<'EOF'
HTTP状态码: %{http_code}\n
DNS解析: %{time_namelookup}s\n
TCP连接: %{time_connect}s\n
TLS握手: %{time_appconnect}s\n
请求开始: %{time_pretransfer}s\n
首字节时间(TTFB): %{time_starttransfer}s\n
总时间: %{time_total}s\n
下载大小: %{size_download} bytes\n
下载速度: %{speed_download} bytes/s\n
EOF
    )
    
    # 记录结束时间
    END_TIME=$(date +%s%N)
    TOTAL_MS=$(echo "scale=0; ($END_TIME - $START_TIME) / 1000000" | bc)
    
    echo "$CURL_OUTPUT"
    echo ""
    
    # 检查响应
    HTTP_CODE=$(echo "$CURL_OUTPUT" | grep "HTTP状态码" | awk '{print $2}')
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ API调用成功${NC}"
        echo "   实际耗时: ${TOTAL_MS}ms"
        
        # 解析响应
        if [ -f /tmp/zhipu_test_response.json ]; then
            TOKENS=$(cat /tmp/zhipu_test_response.json | grep -o '"total_tokens":[0-9]*' | cut -d':' -f2)
            if [ -n "$TOKENS" ]; then
                echo "   使用Token: $TOKENS"
            fi
            
            # 显示响应内容前100字符
            CONTENT=$(cat /tmp/zhipu_test_response.json | head -c 200)
            echo "   响应预览: ${CONTENT}..."
        fi
        
        # 分析时间
        TTFB=$(echo "$CURL_OUTPUT" | grep "首字节时间" | awk '{print $2}' | sed 's/s//')
        TTFB_MS=$(echo "scale=0; $TTFB * 1000" | bc)
        
        if [ $(echo "$TTFB_MS < 1000" | bc) -eq 1 ]; then
            echo -e "   ${GREEN}TTFB评级: 优秀 (<1s)${NC}"
        elif [ $(echo "$TTFB_MS < 3000" | bc) -eq 1 ]; then
            echo -e "   ${YELLOW}TTFB评级: 良好 (<3s)${NC}"
        elif [ $(echo "$TTFB_MS < 5000" | bc) -eq 1 ]; then
            echo -e "   ${YELLOW}TTFB评级: 一般 (<5s)${NC}"
        else
            echo -e "   ${RED}TTFB评级: 较差 (>5s)${NC}"
            echo -e "   ${RED}⚠️  首字节时间过长，这是主要瓶颈！${NC}"
        fi
    else
        echo -e "${RED}❌ API调用失败${NC}"
        echo "   HTTP状态码: $HTTP_CODE"
        if [ -f /tmp/zhipu_test_response.json ]; then
            echo "   错误详情:"
            cat /tmp/zhipu_test_response.json
        fi
    fi
    
    # 清理临时文件
    rm -f /tmp/zhipu_test_request.json /tmp/zhipu_test_response.json
fi
echo ""

# 5. 检查系统资源
echo "5️⃣ 系统资源检查"
echo "----------------------------------------"

# CPU负载
LOAD=$(uptime | awk -F'load averages:' '{print $2}')
echo "CPU负载: $LOAD"

# 内存使用
MEM_INFO=$(vm_stat | perl -ne '/page size of (\d+)/ and $size=$1; /Pages\s+([^:]+)[^\d]+(\d+)/ and printf("%-16s % 16.2f Mi\n", "$1:", $2 * $size / 1048576);')
echo "内存使用:"
echo "$MEM_INFO" | head -3

# 网络连接数
CONN_COUNT=$(netstat -an | grep ESTABLISHED | wc -l | tr -d ' ')
echo "当前TCP连接数: $CONN_COUNT"
echo ""

# 6. 建议
echo "========================================"
echo "💡 诊断建议"
echo "========================================"
echo ""

# 基于测试结果给出建议
if [ -n "$PING_RESULT" ]; then
    PING_INT=$(echo $PING_RESULT | cut -d'.' -f1)
    if [ "$PING_INT" -gt 200 ]; then
        echo -e "${YELLOW}⚠️  网络延迟较高 (${PING_RESULT}ms)${NC}"
        echo "   建议:"
        echo "   1. 检查网络连接质量"
        echo "   2. 考虑使用CDN或代理服务"
        echo "   3. 尝试更换网络环境"
        echo ""
    fi
fi

echo "🔍 主要问题分析:"
echo "   根据日志显示，19.7秒的延迟主要来自："
echo "   1. HTTP请求等待时间 (占比 >99%)"
echo "   2. 不是本地处理问题（内存检索<1ms）"
echo "   3. 不是JSON解析问题（<1ms）"
echo ""

echo "🎯 可能的原因:"
echo "   1. 网络延迟高（需要上面的测试验证）"
echo "   2. Zhipu API服务端处理慢"
echo "   3. API密钥限流"
echo "   4. 使用了非流式模式，需要等待完整响应"
echo ""

echo "✅ 优化建议:"
echo "   1. 启用流式传输（SSE）改善用户体验"
echo "   2. 使用更快的模型（如 glm-4-flash）"
echo "   3. 减少max_tokens限制"
echo "   4. 添加请求超时和重试机制"
echo "   5. 考虑使用缓存减少API调用"
echo "   6. 监控Zhipu API状态页面"
echo ""

echo "📝 下一步:"
echo "   1. 查看上面的测试结果"
echo "   2. 如果TTFB很长（>5s），问题在API端"
echo "   3. 如果网络延迟高（>200ms），问题在网络"
echo "   4. 考虑联系Zhipu技术支持"
echo ""

echo "========================================"
echo "诊断完成！"
echo "========================================"

