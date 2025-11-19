#!/bin/bash
# 华为 MaaS 集成测试脚本

set -e

echo "================================"
echo "华为 MaaS 集成测试"
echo "================================"
echo ""

# 检查环境变量
echo "1. 检查环境变量..."
if [ -z "$MAAS_API_KEY" ] && [ -z "$HUAWEI_MAAS_API_KEY" ]; then
    echo "❌ 错误: 未设置 MAAS_API_KEY 或 HUAWEI_MAAS_API_KEY"
    echo ""
    echo "请设置环境变量:"
    echo "  export MAAS_API_KEY='your_api_key'"
    echo "  export MAAS_MODEL='deepseek-v3.2-exp'  # 可选"
    echo ""
    exit 1
fi

if [ -n "$MAAS_API_KEY" ]; then
    echo "✅ 找到 MAAS_API_KEY"
    MAAS_MODEL=${MAAS_MODEL:-deepseek-v3.2-exp}
elif [ -n "$HUAWEI_MAAS_API_KEY" ]; then
    echo "✅ 找到 HUAWEI_MAAS_API_KEY"
    MAAS_MODEL=${HUAWEI_MAAS_MODEL:-deepseek-v3.2-exp}
fi

echo "   模型: $MAAS_MODEL"
echo ""

# 编译测试
echo "2. 编译 LumosAI 核心..."
cd lumosai/lumosai_core
cargo build --example huawei_maas_agent --release 2>&1 | grep -E "Compiling|Finished" || true
echo "✅ 编译完成"
echo ""

# 运行示例
echo "3. 运行华为 MaaS Agent 示例..."
echo "   (这将调用实际的 API，可能需要几秒钟)"
echo ""

if cargo run --example huawei_maas_agent --release 2>&1; then
    echo ""
    echo "✅ 华为 MaaS 集成测试通过！"
else
    echo ""
    echo "❌ 测试失败，请检查:"
    echo "   1. API Key 是否正确"
    echo "   2. 网络连接是否正常"
    echo "   3. 模型名称是否支持"
    exit 1
fi

echo ""
echo "================================"
echo "测试完成"
echo "================================"
