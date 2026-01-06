#!/bin/bash

# 性能基准测试运行脚本
# 
# 用法:
#   ./scripts/run-benchmarks.sh [options]
#
# 选项:
#   --all             运行所有基准测试
#   --quick           快速测试（减少样本数）
#   --save-baseline   保存基准线
#   --compare         与基准线比较
#   --report          生成 HTML 报告
#   --help            显示此帮助信息

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCHMARK_DIR="${BASE_DIR}/target/criterion"
BASELINE_NAME="baseline"

# 函数：打印信息
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# 函数：打印成功
success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# 函数：打印警告
warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# 函数：打印错误
error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 函数：运行基准测试
run_benchmarks() {
    local quick=$1
    local save_baseline=$2
    local compare=$3
    
    info "Running performance benchmarks..."
    
    cd "${BASE_DIR}"
    
    # 构建基准测试命令
    local cmd="cargo bench --package agent-mem-server"
    
    if [ "${quick}" = "true" ]; then
        info "Running quick benchmarks (reduced sample size)..."
        cmd="${cmd} -- --quick"
    fi
    
    if [ "${save_baseline}" = "true" ]; then
        info "Saving baseline: ${BASELINE_NAME}"
        cmd="${cmd} -- --save-baseline ${BASELINE_NAME}"
    fi
    
    if [ "${compare}" = "true" ]; then
        info "Comparing with baseline: ${BASELINE_NAME}"
        cmd="${cmd} -- --baseline ${BASELINE_NAME}"
    fi
    
    # 运行基准测试
    eval "${cmd}"
    
    if [ $? -eq 0 ]; then
        success "Benchmarks completed successfully!"
        return 0
    else
        error "Benchmarks failed"
        return 1
    fi
}

# 函数：生成 HTML 报告
generate_report() {
    info "Generating HTML report..."
    
    if [ ! -d "${BENCHMARK_DIR}" ]; then
        error "Benchmark results not found. Run benchmarks first."
        return 1
    fi
    
    # Criterion 自动生成 HTML 报告
    info "HTML reports are available at:"
    echo "  ${BENCHMARK_DIR}/report/index.html"
    
    # 尝试在浏览器中打开报告
    if command -v open &> /dev/null; then
        open "${BENCHMARK_DIR}/report/index.html"
        success "Report opened in browser"
    elif command -v xdg-open &> /dev/null; then
        xdg-open "${BENCHMARK_DIR}/report/index.html"
        success "Report opened in browser"
    else
        info "Please open the report manually in your browser"
    fi
}

# 函数：显示基准测试结果摘要
show_summary() {
    info "Benchmark Results Summary"
    echo ""
    
    if [ ! -d "${BENCHMARK_DIR}" ]; then
        warning "No benchmark results found"
        return 1
    fi
    
    # 查找最新的基准测试结果
    local latest_results=$(find "${BENCHMARK_DIR}" -name "estimates.json" | head -5)
    
    if [ -z "${latest_results}" ]; then
        warning "No benchmark results found"
        return 1
    fi
    
    echo "Latest benchmark results:"
    echo "${latest_results}" | while read -r file; do
        local bench_name=$(dirname "${file}" | xargs basename)
        echo "  - ${bench_name}"
    done
    
    echo ""
    info "For detailed results, see: ${BENCHMARK_DIR}/report/index.html"
}

# 函数：清理基准测试结果
clean_results() {
    info "Cleaning benchmark results..."
    
    if [ -d "${BENCHMARK_DIR}" ]; then
        rm -rf "${BENCHMARK_DIR}"
        success "Benchmark results cleaned"
    else
        info "No benchmark results to clean"
    fi
}

# 函数：显示帮助
show_help() {
    cat << EOF
性能基准测试运行脚本

用法:
  $0 [options]

选项:
  --all             运行所有基准测试
  --quick           快速测试（减少样本数）
  --save-baseline   保存基准线
  --compare         与基准线比较
  --report          生成 HTML 报告
  --clean           清理基准测试结果
  --summary         显示结果摘要
  --help            显示此帮助信息

示例:
  # 运行所有基准测试
  $0 --all

  # 快速测试
  $0 --quick

  # 保存基准线
  $0 --save-baseline

  # 与基准线比较
  $0 --compare

  # 生成报告
  $0 --report

  # 清理结果
  $0 --clean

基准测试类别:
  - JSON 序列化/反序列化
  - 字符串操作
  - 集合操作 (Vec, HashMap)
  - 内存分配
  - 并发操作

报告位置:
  ${BENCHMARK_DIR}/report/index.html
EOF
}

# 主函数
main() {
    local run_all=false
    local quick=false
    local save_baseline=false
    local compare=false
    local generate_report_flag=false
    local clean_flag=false
    local summary_flag=false
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                run_all=true
                shift
                ;;
            --quick)
                quick=true
                run_all=true
                shift
                ;;
            --save-baseline)
                save_baseline=true
                run_all=true
                shift
                ;;
            --compare)
                compare=true
                run_all=true
                shift
                ;;
            --report)
                generate_report_flag=true
                shift
                ;;
            --clean)
                clean_flag=true
                shift
                ;;
            --summary)
                summary_flag=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 执行操作
    if [ "${clean_flag}" = "true" ]; then
        clean_results
        exit 0
    fi
    
    if [ "${summary_flag}" = "true" ]; then
        show_summary
        exit 0
    fi
    
    if [ "${run_all}" = "true" ]; then
        run_benchmarks "${quick}" "${save_baseline}" "${compare}"
        BENCH_RESULT=$?
        
        if [ "${generate_report_flag}" = "true" ]; then
            generate_report
        fi
        
        show_summary
        exit ${BENCH_RESULT}
    fi
    
    if [ "${generate_report_flag}" = "true" ]; then
        generate_report
        exit 0
    fi
    
    # 默认：显示帮助
    show_help
}

# 运行主函数
main "$@"

