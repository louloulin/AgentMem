#!/bin/bash

# LLM 记忆效果全面分析演示运行脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}=== AgentMem LLM 记忆效果全面分析 ===${NC}"
echo ""

# 显示帮助信息
show_help() {
    echo "用法: ./run.sh [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help          显示帮助信息"
    echo "  -v, --verbose       显示详细日志"
    echo "  --deepseek KEY      使用 DeepSeek（指定 API Key）"
    echo ""
    echo "示例:"
    echo "  ./run.sh --deepseek sk-xxx"
    echo "  ./run.sh -v"
    echo ""
}

# 解析命令行参数
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --deepseek)
            export DEEPSEEK_API_KEY="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}❌ 未知选项: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 检查 LLM 配置
echo -e "${BLUE}📋 检查 LLM 配置...${NC}"
echo ""

if [ -n "$DEEPSEEK_API_KEY" ]; then
    echo -e "${GREEN}✅ 检测到 DeepSeek API Key${NC}"
    echo "   使用 DeepSeek 作为 LLM 提供商"
    echo ""
else
    echo -e "${RED}❌ 未检测到 DeepSeek API Key${NC}"
    echo ""
    echo "请设置 DEEPSEEK_API_KEY 环境变量："
    echo "  export DEEPSEEK_API_KEY=your_api_key"
    echo "  ./run.sh"
    echo ""
    exit 1
fi

# 设置日志级别
if [ "$VERBOSE" = true ]; then
    export RUST_LOG="info,agent_mem_llm=debug"
else
    export RUST_LOG="info"
fi

# 运行演示
echo -e "${CYAN}🚀 开始运行演示...${NC}"
echo ""

START_TIME=$(date +%s)

cargo run --package llm-memory-analysis-demo --release

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo ""
echo -e "${GREEN}✅ 演示完成！${NC}"
echo ""
echo -e "${BLUE}📈 统计信息${NC}"
echo "  总耗时: ${ELAPSED} 秒"
echo ""
echo -e "${CYAN}📚 查看详细报告${NC}"
echo "  项目总结: cat SUMMARY.md"
echo "  性能分析: cat PERFORMANCE_ANALYSIS.md"
echo ""
