#!/bin/bash
# 设置 ONNX Runtime 库路径并运行测试

# 设置项目根目录
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"

# 设置 ONNX Runtime 库路径（使用项目中的 1.22.0 版本）
export DYLD_LIBRARY_PATH="$PROJECT_ROOT/lib:$DYLD_LIBRARY_PATH"
export LD_LIBRARY_PATH="$PROJECT_ROOT/lib:$LD_LIBRARY_PATH"

# 打印环境变量
echo "========================================="
echo "ONNX Runtime 环境配置"
echo "========================================="
echo "DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"
echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
echo ""
echo "ONNX Runtime 库文件:"
ls -lh "$PROJECT_ROOT/lib/libonnxruntime.dylib"
echo ""
echo "ONNX Runtime 版本:"
otool -L "$PROJECT_ROOT/lib/libonnxruntime.dylib" | grep onnxruntime
echo "========================================="
echo ""

# 运行测试
echo "开始运行测试..."
cargo test --workspace --lib --release "$@"

