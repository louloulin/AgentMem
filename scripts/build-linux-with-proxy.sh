#!/bin/bash
# 快速设置代理并构建 Linux 版本的便捷脚本

# 默认代理地址（请根据实际情况修改）
DEFAULT_PROXY="${DEFAULT_PROXY:-http://127.0.0.1:4780}"

# 使用方法提示
show_usage() {
    echo "使用方法:"
    echo "  $0 [代理地址]"
    echo ""
    echo "示例:"
    echo "  $0                                    # 使用默认代理: $DEFAULT_PROXY"
    echo "  $0 http://127.0.0.1:7890            # 使用指定 HTTP 代理"
    echo "  $0 socks5://127.0.0.1:7890         # 使用 SOCKS5 代理"
    echo "  $0 --system                         # 使用系统环境变量中的代理"
    echo "  $0 --no-proxy                       # 不使用代理"
    echo ""
}

# 解析参数
PROXY_ARG="${1:-$DEFAULT_PROXY}"

if [ "$PROXY_ARG" = "--help" ] || [ "$PROXY_ARG" = "-h" ]; then
    show_usage
    exit 0
fi

if [ "$PROXY_ARG" = "--system" ]; then
    echo "使用系统环境变量中的代理设置..."
    bash scripts/build-linux-cross.sh --proxy-system release
elif [ "$PROXY_ARG" = "--no-proxy" ]; then
    echo "不使用代理，直接构建..."
    unset HTTP_PROXY HTTPS_PROXY ALL_PROXY
    bash scripts/build-linux-cross.sh release
else
    echo "设置代理: $PROXY_ARG"
    export HTTP_PROXY="$PROXY_ARG"
    export HTTPS_PROXY="$PROXY_ARG"
    if [[ "$PROXY_ARG" == socks5://* ]]; then
        export ALL_PROXY="$PROXY_ARG"
    fi
    bash scripts/build-linux-cross.sh --proxy "$PROXY_ARG" release
fi
