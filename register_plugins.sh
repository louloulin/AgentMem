#!/bin/bash
# 注册所有编译好的插件

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_URL="http://localhost:8080/api/v1/plugins"
WASM_DIR="$SCRIPT_DIR/target/wasm32-wasip1/release"
USER_ID="default"
ORG_ID="default-org"

echo "🔌 开始注册插件..."
echo ""

# 1. Hello Plugin
echo "📦 注册 Hello Plugin..."
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID" \
  -d '{
    "id": "hello-plugin",
    "metadata": {
      "name": "Hello Plugin",
      "version": "1.0.0",
      "description": "A simple hello world plugin demonstrating basic WASM functionality",
      "author": "AgentMem Team",
      "plugin_type": "memory_processor",
      "required_capabilities": ["memory_access", "logging_access"]
    },
    "path": "'"$WASM_DIR/hello_plugin.wasm"'",
    "config": {}
  }' > /tmp/plugin_reg_1.json
echo "响应:" && cat /tmp/plugin_reg_1.json | jq '.' && echo ""

# 2. Memory Processor Plugin
echo "📦 注册 Memory Processor Plugin..."
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID" \
  -d '{
    "id": "memory-processor",
    "metadata": {
      "name": "Memory Processor",
      "version": "1.0.0",
      "description": "Advanced memory content processor with keyword extraction and summarization",
      "author": "AgentMem Team",
      "plugin_type": "memory_processor",
      "required_capabilities": ["memory_access", "logging_access", "storage_access"]
    },
    "path": "'"$WASM_DIR/memory_processor_plugin.wasm"'",
    "config": {}
  }' > /tmp/plugin_reg_2.json
echo "响应:" && cat /tmp/plugin_reg_2.json | jq '.' && echo ""

# 3. Code Analyzer Plugin
echo "📦 注册 Code Analyzer Plugin..."
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID" \
  -d '{
    "id": "code-analyzer",
    "metadata": {
      "name": "Code Analyzer",
      "version": "1.0.0",
      "description": "Static code analyzer for Rust and Python with complexity metrics",
      "author": "AgentMem Team",
      "plugin_type": "memory_processor",
      "required_capabilities": ["memory_access", "logging_access"]
    },
    "path": "'"$WASM_DIR/code_analyzer_plugin.wasm"'",
    "config": {}
  }' > /tmp/plugin_reg_3.json
echo "响应:" && cat /tmp/plugin_reg_3.json | jq '.' && echo ""

# 4. LLM Plugin
echo "📦 注册 LLM Plugin..."
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID" \
  -d '{
    "id": "llm-plugin",
    "metadata": {
      "name": "LLM Plugin",
      "version": "1.0.0",
      "description": "LLM integration plugin for text summarization, translation and Q&A",
      "author": "AgentMem Team",
      "plugin_type": "llm_integration",
      "required_capabilities": ["llm_access", "logging_access"]
    },
    "path": "'"$WASM_DIR/llm_plugin.wasm"'",
    "config": {}
  }' > /tmp/plugin_reg_4.json
echo "响应:" && cat /tmp/plugin_reg_4.json | jq '.' && echo ""

echo ""
echo "=== ✅ 插件注册完成 ==="
echo ""
echo "📊 查看已注册插件："
curl -s "$API_URL" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID" | jq '.'
