#!/bin/bash
# 配置代理设置

# 设置 HTTPS 代理
export https_proxy="http://127.0.0.1:4780"

# 设置 HTTP 代理（如果需要）
export http_proxy="http://127.0.0.1:4780"

# 显示配置
echo "✅ 代理配置已设置:"
echo "  https_proxy=$https_proxy"
echo "  http_proxy=$http_proxy"

# 验证代理（可选）
echo ""
echo "测试代理连接..."
if curl -s --proxy "$https_proxy" https://www.google.com > /dev/null 2>&1; then
    echo "✅ 代理连接成功"
else
    echo "⚠️  代理连接测试失败（可能代理未启动或配置不正确）"
fi
