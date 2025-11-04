#!/bin/bash
# Build all WASM plugins and copy to target directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="$SCRIPT_DIR/target/wasm32-wasip1/release"

echo "=== Building WASM Plugins ==="
echo ""

# Create target directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Build hello_plugin
echo "📦 Building hello_plugin..."
cd "$SCRIPT_DIR/crates/agent-mem-plugin-sdk/examples/hello_plugin"
cargo build --target wasm32-wasip1 --release --quiet
cp target/wasm32-wasip1/release/hello_plugin.wasm "$TARGET_DIR/"
echo "   ✅ hello_plugin.wasm ($(du -h "$TARGET_DIR/hello_plugin.wasm" | cut -f1))"

# Build memory_processor
echo "📦 Building memory_processor..."
cd "$SCRIPT_DIR/crates/agent-mem-plugin-sdk/examples/memory_processor"
cargo build --target wasm32-wasip1 --release --quiet
cp target/wasm32-wasip1/release/memory_processor_plugin.wasm "$TARGET_DIR/"
echo "   ✅ memory_processor_plugin.wasm ($(du -h "$TARGET_DIR/memory_processor_plugin.wasm" | cut -f1))"

# Build code_analyzer
echo "📦 Building code_analyzer..."
cd "$SCRIPT_DIR/crates/agent-mem-plugin-sdk/examples/code_analyzer"
cargo build --target wasm32-wasip1 --release --quiet
cp target/wasm32-wasip1/release/code_analyzer_plugin.wasm "$TARGET_DIR/"
echo "   ✅ code_analyzer_plugin.wasm ($(du -h "$TARGET_DIR/code_analyzer_plugin.wasm" | cut -f1))"

echo ""
echo "=== ✅ All plugins built successfully! ==="
echo ""
echo "📁 WASM files location: $TARGET_DIR"
ls -lh "$TARGET_DIR"/*.wasm 2>/dev/null || true

cd "$SCRIPT_DIR"

